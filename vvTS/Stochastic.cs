using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000087 RID: 135
	[HandlerCategory("vvStoch"), HandlerName("Stochastic")]
	public class Stochastic : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060004AC RID: 1196 RVA: 0x00017E94 File Offset: 0x00016094
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("stochastic", new string[]
			{
				this.Kperiod.ToString(),
				this.D.ToString(),
				this.postSmooth.ToString(),
				this.SmoothPhase.ToString(),
				sec.get_CacheName()
			}, () => Stochastic.GenStochastic(sec, this.Kperiod, this.D, this.postSmooth, this.SmoothPhase, this.Context));
		}

		// Token: 0x060004AB RID: 1195 RVA: 0x00017CF0 File Offset: 0x00015EF0
		public static IList<double> GenStochastic(ISecurity _sec, int kperiod, bool _D, int postsmooth, int _SmoothPhase, IContext _ctx)
		{
			IList<double> data = _ctx.GetData("hhv", new string[]
			{
				kperiod.ToString(),
				_sec.get_CacheName()
			}, () => Series.Highest(_sec.get_HighPrices(), kperiod));
			IList<double> data2 = _ctx.GetData("llv", new string[]
			{
				kperiod.ToString(),
				_sec.get_CacheName()
			}, () => Series.Lowest(_sec.get_LowPrices(), kperiod));
			IList<double> closePrices = _sec.get_ClosePrices();
			double[] array = new double[closePrices.Count];
			for (int i = 0; i < closePrices.Count; i++)
			{
				double num = data[i] - data2[i];
				array[i] = ((num == 0.0) ? 0.0 : (100.0 * (closePrices[i] - data2[i]) / num));
			}
			IList<double> list = Series.SMA(array, 3);
			if (postsmooth > 0)
			{
				return JMA.GenJMA(_D ? list : array, postsmooth, _SmoothPhase);
			}
			if (!_D)
			{
				return array;
			}
			return list;
		}

		// Token: 0x17000198 RID: 408
		public IContext Context
		{
			// Token: 0x060004AD RID: 1197 RVA: 0x00017F2D File Offset: 0x0001612D
			get;
			// Token: 0x060004AE RID: 1198 RVA: 0x00017F35 File Offset: 0x00016135
			set;
		}

		// Token: 0x17000195 RID: 405
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool D
		{
			// Token: 0x060004A5 RID: 1189 RVA: 0x00017C83 File Offset: 0x00015E83
			get;
			// Token: 0x060004A6 RID: 1190 RVA: 0x00017C8B File Offset: 0x00015E8B
			set;
		}

		// Token: 0x17000194 RID: 404
		[HandlerParameter(true, "14", Min = "0", Max = "20", Step = "1")]
		public int Kperiod
		{
			// Token: 0x060004A3 RID: 1187 RVA: 0x00017C72 File Offset: 0x00015E72
			get;
			// Token: 0x060004A4 RID: 1188 RVA: 0x00017C7A File Offset: 0x00015E7A
			set;
		}

		// Token: 0x17000196 RID: 406
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int postSmooth
		{
			// Token: 0x060004A7 RID: 1191 RVA: 0x00017C94 File Offset: 0x00015E94
			get;
			// Token: 0x060004A8 RID: 1192 RVA: 0x00017C9C File Offset: 0x00015E9C
			set;
		}

		// Token: 0x17000197 RID: 407
		[HandlerParameter(true, "100", Min = "-100", Max = "100", Step = "20")]
		public int SmoothPhase
		{
			// Token: 0x060004A9 RID: 1193 RVA: 0x00017CA5 File Offset: 0x00015EA5
			get;
			// Token: 0x060004AA RID: 1194 RVA: 0x00017CAD File Offset: 0x00015EAD
			set;
		}
	}
}
