using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200008E RID: 142
	[HandlerCategory("vvStoch"), HandlerName("StochK")]
	public class StochK : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060004EE RID: 1262 RVA: 0x000192D4 File Offset: 0x000174D4
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("StochK", new string[]
			{
				this.Kperiod.ToString(),
				this.postSmooth.ToString(),
				sec.get_CacheName()
			}, () => StochK.GenStochK(sec, this.Kperiod, this.postSmooth, this.Context));
		}

		// Token: 0x060004ED RID: 1261 RVA: 0x00019168 File Offset: 0x00017368
		public static IList<double> GenStochK(ISecurity _sec, int _kperiod, int _postsmooth, IContext _ctx)
		{
			IList<double> data = _ctx.GetData("hhv", new string[]
			{
				_kperiod.ToString(),
				_sec.get_CacheName()
			}, () => Series.Highest(_sec.get_HighPrices(), _kperiod));
			IList<double> data2 = _ctx.GetData("llv", new string[]
			{
				_kperiod.ToString(),
				_sec.get_CacheName()
			}, () => Series.Lowest(_sec.get_LowPrices(), _kperiod));
			IList<double> closePrices = _sec.get_ClosePrices();
			double[] array = new double[closePrices.Count];
			for (int i = 0; i < closePrices.Count; i++)
			{
				double num = data[i] - data2[i];
				array[i] = ((num == 0.0) ? 0.0 : (100.0 * (closePrices[i] - data2[i]) / num));
			}
			if (_postsmooth > 0)
			{
				return JMA.GenJMA(array, _postsmooth, 100);
			}
			return array;
		}

		// Token: 0x170001AF RID: 431
		public IContext Context
		{
			// Token: 0x060004EF RID: 1263 RVA: 0x00019349 File Offset: 0x00017549
			get;
			// Token: 0x060004F0 RID: 1264 RVA: 0x00019351 File Offset: 0x00017551
			set;
		}

		// Token: 0x170001AD RID: 429
		[HandlerParameter(true, "14", Min = "0", Max = "20", Step = "1")]
		public int Kperiod
		{
			// Token: 0x060004E9 RID: 1257 RVA: 0x0001910B File Offset: 0x0001730B
			get;
			// Token: 0x060004EA RID: 1258 RVA: 0x00019113 File Offset: 0x00017313
			set;
		}

		// Token: 0x170001AE RID: 430
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int postSmooth
		{
			// Token: 0x060004EB RID: 1259 RVA: 0x0001911C File Offset: 0x0001731C
			get;
			// Token: 0x060004EC RID: 1260 RVA: 0x00019124 File Offset: 0x00017324
			set;
		}
	}
}
