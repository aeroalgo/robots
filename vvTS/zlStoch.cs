using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000094 RID: 148
	[HandlerCategory("vvStoch"), HandlerName("ZeroLag Stochastic")]
	public class zlStoch : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000540 RID: 1344 RVA: 0x0001A843 File Offset: 0x00018A43
		public IList<double> Execute(ISecurity sec)
		{
			return this.GenzlStoch(sec, this.Kperiod, this.Dperiod, this.Slowing);
		}

		// Token: 0x0600053F RID: 1343 RVA: 0x0001A640 File Offset: 0x00018840
		public IList<double> GenzlStoch(ISecurity _sec, int _Kperiod, int _Dperiod, int _Slowing)
		{
			IList<double> closePrices = _sec.get_ClosePrices();
			int arg_2A_0 = closePrices.Count;
			IList<double> data = this.Context.GetData("hhv", new string[]
			{
				_Kperiod.ToString(),
				_sec.get_CacheName()
			}, () => Series.Highest(_sec.get_HighPrices(), _Kperiod));
			IList<double> data2 = this.Context.GetData("llv", new string[]
			{
				_Kperiod.ToString(),
				_sec.get_CacheName()
			}, () => Series.Lowest(_sec.get_LowPrices(), _Kperiod));
			double[] array = new double[closePrices.Count];
			double[] array2 = new double[closePrices.Count];
			double[] array3 = new double[closePrices.Count];
			for (int i = 0; i < closePrices.Count; i++)
			{
				double num = data[i] - data2[i];
				array[i] = ((num == 0.0) ? 0.0 : (100.0 * ((closePrices[i] - data2[i]) / (data[i] - data2[i]))));
			}
			IList<double> list = Series.SMA(array, _Slowing);
			IList<double> list2 = Series.EMA(list, _Slowing);
			for (int j = 0; j < closePrices.Count; j++)
			{
				array2[j] = 2.0 * list[j] - list2[j];
			}
			IList<double> list3 = Series.SMA(array2, _Dperiod);
			IList<double> list4 = Series.SMA(list3, _Dperiod);
			for (int k = 0; k < closePrices.Count; k++)
			{
				array3[k] = 2.0 * list3[k] - list4[k];
			}
			if (!this.SignalLine)
			{
				return array2;
			}
			return array3;
		}

		// Token: 0x170001CE RID: 462
		public IContext Context
		{
			// Token: 0x06000541 RID: 1345 RVA: 0x0001A85E File Offset: 0x00018A5E
			get;
			// Token: 0x06000542 RID: 1346 RVA: 0x0001A866 File Offset: 0x00018A66
			set;
		}

		// Token: 0x170001CB RID: 459
		[HandlerParameter(true, "3", Min = "1", Max = "20", Step = "1")]
		public int Dperiod
		{
			// Token: 0x06000539 RID: 1337 RVA: 0x0001A5D5 File Offset: 0x000187D5
			get;
			// Token: 0x0600053A RID: 1338 RVA: 0x0001A5DD File Offset: 0x000187DD
			set;
		}

		// Token: 0x170001CA RID: 458
		[HandlerParameter(true, "5", Min = "1", Max = "10", Step = "1")]
		public int Kperiod
		{
			// Token: 0x06000537 RID: 1335 RVA: 0x0001A5C4 File Offset: 0x000187C4
			get;
			// Token: 0x06000538 RID: 1336 RVA: 0x0001A5CC File Offset: 0x000187CC
			set;
		}

		// Token: 0x170001CD RID: 461
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool SignalLine
		{
			// Token: 0x0600053D RID: 1341 RVA: 0x0001A5F7 File Offset: 0x000187F7
			get;
			// Token: 0x0600053E RID: 1342 RVA: 0x0001A5FF File Offset: 0x000187FF
			set;
		}

		// Token: 0x170001CC RID: 460
		[HandlerParameter(true, "3", Min = "1", Max = "10", Step = "1")]
		public int Slowing
		{
			// Token: 0x0600053B RID: 1339 RVA: 0x0001A5E6 File Offset: 0x000187E6
			get;
			// Token: 0x0600053C RID: 1340 RVA: 0x0001A5EE File Offset: 0x000187EE
			set;
		}
	}
}
