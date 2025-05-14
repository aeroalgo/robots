using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200008C RID: 140
	[HandlerCategory("vvStoch"), HandlerName("SlowStoch")]
	public class SlowStoch : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060004DC RID: 1244 RVA: 0x00018E08 File Offset: 0x00017008
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("slowstoch", new string[]
			{
				this.Kperiod.ToString(),
				this.D.ToString(),
				sec.get_CacheName()
			}, () => SlowStoch.GenSlowStoch(sec, this.Context, this.Kperiod, this.D));
		}

		// Token: 0x060004DB RID: 1243 RVA: 0x00018C88 File Offset: 0x00016E88
		public static IList<double> GenSlowStoch(ISecurity src, IContext ctx, int kperiod, bool _D)
		{
			IList<double> data = ctx.GetData("hhv", new string[]
			{
				kperiod.ToString(),
				src.get_CacheName()
			}, () => Series.Highest(src.get_HighPrices(), kperiod));
			IList<double> data2 = ctx.GetData("llv", new string[]
			{
				kperiod.ToString(),
				src.get_CacheName()
			}, () => Series.Lowest(src.get_LowPrices(), kperiod));
			IList<double> closePrices = src.get_ClosePrices();
			double[] array = new double[closePrices.Count];
			for (int i = 0; i < closePrices.Count; i++)
			{
				double num = data[i] - data2[i];
				array[i] = ((num == 0.0) ? 0.0 : (100.0 * (closePrices[i] - data2[i]) / num));
			}
			SMA.GenSMA(array, 3);
			IList<double> list = SMA.GenSMA(array, 3);
			IList<double> result = SMA.GenSMA(list, 3);
			if (!_D)
			{
				return list;
			}
			return result;
		}

		// Token: 0x170001A9 RID: 425
		public IContext Context
		{
			// Token: 0x060004DD RID: 1245 RVA: 0x00018E7D File Offset: 0x0001707D
			get;
			// Token: 0x060004DE RID: 1246 RVA: 0x00018E85 File Offset: 0x00017085
			set;
		}

		// Token: 0x170001A8 RID: 424
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool D
		{
			// Token: 0x060004D9 RID: 1241 RVA: 0x00018C3D File Offset: 0x00016E3D
			get;
			// Token: 0x060004DA RID: 1242 RVA: 0x00018C45 File Offset: 0x00016E45
			set;
		}

		// Token: 0x170001A7 RID: 423
		[HandlerParameter(true, "14", Min = "0", Max = "20", Step = "1")]
		public int Kperiod
		{
			// Token: 0x060004D7 RID: 1239 RVA: 0x00018C2C File Offset: 0x00016E2C
			get;
			// Token: 0x060004D8 RID: 1240 RVA: 0x00018C34 File Offset: 0x00016E34
			set;
		}
	}
}
