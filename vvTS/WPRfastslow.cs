using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000078 RID: 120
	[HandlerCategory("vvWilliams"), HandlerName("WPRfast(-slow)")]
	public class WPRfastslow : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000447 RID: 1095 RVA: 0x00016BDD File Offset: 0x00014DDD
		public IList<double> Execute(ISecurity sec)
		{
			return WPRfastslow.GenWPRfastslow(sec, this.Period, this.n1, this.n2, this.Context);
		}

		// Token: 0x06000446 RID: 1094 RVA: 0x00016A1C File Offset: 0x00014C1C
		public static IList<double> GenWPRfastslow(ISecurity _sec, int _period, int _n1, int _n2, IContext _ctx)
		{
			IList<double> data = _ctx.GetData("hhv", new string[]
			{
				_period.ToString(),
				_sec.get_CacheName()
			}, () => Series.Highest(_sec.get_HighPrices(), _period));
			IList<double> data2 = _ctx.GetData("llv", new string[]
			{
				_period.ToString(),
				_sec.get_CacheName()
			}, () => Series.Lowest(_sec.get_LowPrices(), _period));
			IList<double> closePrices = _sec.get_ClosePrices();
			double[] array = new double[closePrices.Count];
			double[] array2 = new double[closePrices.Count];
			for (int i = 0; i < closePrices.Count; i++)
			{
				if (data[i] - data2[i] != 0.0)
				{
					array[i] = -(data[i] - closePrices[i]) / (data[i] - data2[i]) * 100.0;
				}
				else
				{
					array[i] = -(data[i] - closePrices[i]) / (data[i] - data2[i] + 1E-10) * 100.0;
				}
				double a = array[i] * array[i] / 100.0;
				double num = Math.Ceiling(a);
				if (num < (double)_n1)
				{
					array2[i] = 1.0;
				}
				if (num > (double)_n2)
				{
					array2[i] = -1.0;
				}
			}
			return array2;
		}

		// Token: 0x17000174 RID: 372
		public IContext Context
		{
			// Token: 0x06000448 RID: 1096 RVA: 0x00016BFD File Offset: 0x00014DFD
			get;
			// Token: 0x06000449 RID: 1097 RVA: 0x00016C05 File Offset: 0x00014E05
			set;
		}

		// Token: 0x17000172 RID: 370
		[HandlerParameter(true, "9", Min = "1", Max = "50", Step = "1")]
		public int n1
		{
			// Token: 0x06000442 RID: 1090 RVA: 0x000169C0 File Offset: 0x00014BC0
			get;
			// Token: 0x06000443 RID: 1091 RVA: 0x000169C8 File Offset: 0x00014BC8
			set;
		}

		// Token: 0x17000173 RID: 371
		[HandlerParameter(true, "49", Min = "1", Max = "50", Step = "1")]
		public int n2
		{
			// Token: 0x06000444 RID: 1092 RVA: 0x000169D1 File Offset: 0x00014BD1
			get;
			// Token: 0x06000445 RID: 1093 RVA: 0x000169D9 File Offset: 0x00014BD9
			set;
		}

		// Token: 0x17000171 RID: 369
		[HandlerParameter]
		public int Period
		{
			// Token: 0x06000440 RID: 1088 RVA: 0x000169AF File Offset: 0x00014BAF
			get;
			// Token: 0x06000441 RID: 1089 RVA: 0x000169B7 File Offset: 0x00014BB7
			set;
		}
	}
}
