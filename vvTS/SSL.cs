using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000059 RID: 89
	[HandlerCategory("vvIndicators"), HandlerName("SSL (Gann high-low activator)")]
	public class SSL : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600032F RID: 815 RVA: 0x000125E0 File Offset: 0x000107E0
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("GHLA", new string[]
			{
				this.Lb.ToString(),
				sec.get_CacheName()
			}, () => SSL.GenSSL(sec, this.Context, this.Lb));
		}

		// Token: 0x0600032E RID: 814 RVA: 0x00012414 File Offset: 0x00010614
		public static IList<double> GenSSL(ISecurity _sec, IContext _ctx, int _Lb)
		{
			double num = 0.0;
			double[] array = new double[_sec.get_ClosePrices().Count];
			IList<double> closePrices = _sec.get_ClosePrices();
			IList<double> data = _ctx.GetData("sma", new string[]
			{
				_Lb.ToString(),
				_sec.get_HighPrices().GetHashCode().ToString()
			}, () => Series.SMA(_sec.get_HighPrices(), _Lb));
			IList<double> data2 = _ctx.GetData("sma", new string[]
			{
				_Lb.ToString(),
				_sec.get_LowPrices().GetHashCode().ToString()
			}, () => Series.SMA(_sec.get_LowPrices(), _Lb));
			for (int i = 1; i < _sec.get_ClosePrices().Count; i++)
			{
				double num2;
				if (closePrices[i] > data[i])
				{
					num2 = 1.0;
				}
				else if (closePrices[i] < data2[i])
				{
					num2 = -1.0;
				}
				else
				{
					num2 = 0.0;
				}
				if (num2 != 0.0)
				{
					num = num2;
				}
				if (num == -1.0)
				{
					array[i] = data[i - 1];
				}
				else
				{
					array[i] = data2[i - 1];
				}
			}
			return array;
		}

		// Token: 0x17000113 RID: 275
		public IContext Context
		{
			// Token: 0x06000330 RID: 816 RVA: 0x00012644 File Offset: 0x00010844
			get;
			// Token: 0x06000331 RID: 817 RVA: 0x0001264C File Offset: 0x0001084C
			set;
		}

		// Token: 0x17000112 RID: 274
		[HandlerParameter(true, "10", Min = "1", Max = "100", Step = "1")]
		public int Lb
		{
			// Token: 0x0600032C RID: 812 RVA: 0x000123CB File Offset: 0x000105CB
			get;
			// Token: 0x0600032D RID: 813 RVA: 0x000123D3 File Offset: 0x000105D3
			set;
		}
	}
}
