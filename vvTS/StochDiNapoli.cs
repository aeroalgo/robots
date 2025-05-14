using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200008F RID: 143
	[HandlerCategory("vvStoch"), HandlerName("StochDiNapoli")]
	public class StochDiNapoli : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060004FC RID: 1276 RVA: 0x00019746 File Offset: 0x00017946
		public IList<double> Execute(ISecurity sec)
		{
			return this.GenStochDiNapoli(sec, this.FastK, this.Context);
		}

		// Token: 0x060004FD RID: 1277 RVA: 0x0001975B File Offset: 0x0001795B
		public IList<double> Execute(IList<double> src)
		{
			return this.GenStochDiNapoli(src, this.FastK, this.Context);
		}

		// Token: 0x060004FA RID: 1274 RVA: 0x000193E0 File Offset: 0x000175E0
		public IList<double> GenStochDiNapoli(ISecurity _sec, int _period, IContext _ctx)
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
			double[] array3 = new double[closePrices.Count];
			for (int i = 1; i < closePrices.Count; i++)
			{
				double num = data[i] - data2[i];
				array[i] = ((num == 0.0) ? 0.0 : (100.0 * (closePrices[i] - data2[i]) / num));
				array2[i] = array2[i - 1] + (array[i] - array2[i - 1]) / (double)this.SlowK;
				array3[i] = array3[i - 1] + (array2[i] - array3[i - 1]) / (double)this.SlowD;
			}
			if (!this.SignalLine)
			{
				return array2;
			}
			return array3;
		}

		// Token: 0x060004FB RID: 1275 RVA: 0x000195A4 File Offset: 0x000177A4
		public IList<double> GenStochDiNapoli(IList<double> src, int _period, IContext _ctx)
		{
			IList<double> data = _ctx.GetData("hhv", new string[]
			{
				_period.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.Highest(src, _period));
			IList<double> data2 = _ctx.GetData("llv", new string[]
			{
				_period.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.Lowest(src, _period));
			IList<double> src2 = src;
			double[] array = new double[src2.Count];
			double[] array2 = new double[src2.Count];
			double[] array3 = new double[src2.Count];
			for (int i = 1; i < src2.Count; i++)
			{
				double num = data[i] - data2[i];
				array[i] = ((num == 0.0) ? 0.0 : (100.0 * (src2[i] - data2[i]) / num));
				array2[i] = array2[i - 1] + (array[i] - array2[i - 1]) / (double)this.SlowK;
				array3[i] = array3[i - 1] + (array2[i] - array3[i - 1]) / (double)this.SlowD;
			}
			if (!this.SignalLine)
			{
				return array2;
			}
			return array3;
		}

		// Token: 0x170001B4 RID: 436
		public IContext Context
		{
			// Token: 0x060004FE RID: 1278 RVA: 0x00019770 File Offset: 0x00017970
			get;
			// Token: 0x060004FF RID: 1279 RVA: 0x00019778 File Offset: 0x00017978
			set;
		}

		// Token: 0x170001B0 RID: 432
		[HandlerParameter(true, "8", Min = "1", Max = "30", Step = "1")]
		public int FastK
		{
			// Token: 0x060004F2 RID: 1266 RVA: 0x00019362 File Offset: 0x00017562
			get;
			// Token: 0x060004F3 RID: 1267 RVA: 0x0001936A File Offset: 0x0001756A
			set;
		}

		// Token: 0x170001B3 RID: 435
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool SignalLine
		{
			// Token: 0x060004F8 RID: 1272 RVA: 0x00019395 File Offset: 0x00017595
			get;
			// Token: 0x060004F9 RID: 1273 RVA: 0x0001939D File Offset: 0x0001759D
			set;
		}

		// Token: 0x170001B2 RID: 434
		[HandlerParameter(true, "3", Min = "1", Max = "10", Step = "1")]
		public int SlowD
		{
			// Token: 0x060004F6 RID: 1270 RVA: 0x00019384 File Offset: 0x00017584
			get;
			// Token: 0x060004F7 RID: 1271 RVA: 0x0001938C File Offset: 0x0001758C
			set;
		}

		// Token: 0x170001B1 RID: 433
		[HandlerParameter(true, "3", Min = "1", Max = "10", Step = "1")]
		public int SlowK
		{
			// Token: 0x060004F4 RID: 1268 RVA: 0x00019373 File Offset: 0x00017573
			get;
			// Token: 0x060004F5 RID: 1269 RVA: 0x0001937B File Offset: 0x0001757B
			set;
		}
	}
}
