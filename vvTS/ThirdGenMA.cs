using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000152 RID: 338
	[HandlerCategory("vvAverages"), HandlerName("3rd Generation MA")]
	public class ThirdGenMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000A96 RID: 2710 RVA: 0x0002BCB0 File Offset: 0x00029EB0
		public IList<double> Execute(IList<double> src)
		{
			int count = src.Count;
			double[] array = new double[count];
			double num = (double)(this.MA_Period / this.MA_Sampling_Period);
			double num2 = num * (double)(this.MA_Period - 1) / ((double)this.MA_Period - num);
			IList<double> data = this.Context.GetData("3dGenMa1", new string[]
			{
				this.MA_Sampling_Period.ToString(),
				this.MA_Period.ToString(),
				this.MA_Method.ToString()
			}, () => AllAverages.Gen_mMA(src, this.Context, this.MA_Method, this.MA_Period, this.MA_Sampling_Period, 0.0, 0.0));
			IList<double> list = AllAverages.Gen_mMA(data, this.Context, this.MA_Method, this.MA_Sampling_Period, this.MA_Sampling_Period / 2, 0.0, 0.0);
			for (int i = 0; i < count; i++)
			{
				array[i] = (num2 + 1.0) * data[i] - num2 * list[i];
			}
			return array;
		}

		// Token: 0x17000381 RID: 897
		public IContext Context
		{
			// Token: 0x06000A97 RID: 2711 RVA: 0x0002BDE0 File Offset: 0x00029FE0
			get;
			// Token: 0x06000A98 RID: 2712 RVA: 0x0002BDE8 File Offset: 0x00029FE8
			set;
		}

		// Token: 0x17000380 RID: 896
		[HandlerParameter(true, "2", Min = "0", Max = "3", Step = "1", Name = "Mode:\n0-sma,2-ema,3-lwma")]
		public int MA_Method
		{
			// Token: 0x06000A94 RID: 2708 RVA: 0x0002BC3C File Offset: 0x00029E3C
			get;
			// Token: 0x06000A95 RID: 2709 RVA: 0x0002BC44 File Offset: 0x00029E44
			set;
		}

		// Token: 0x1700037E RID: 894
		[HandlerParameter(true, "220", Min = "5", Max = "300", Step = "1")]
		public int MA_Period
		{
			// Token: 0x06000A90 RID: 2704 RVA: 0x0002BC1A File Offset: 0x00029E1A
			get;
			// Token: 0x06000A91 RID: 2705 RVA: 0x0002BC22 File Offset: 0x00029E22
			set;
		}

		// Token: 0x1700037F RID: 895
		[HandlerParameter(true, "50", Min = "5", Max = "50", Step = "1")]
		public int MA_Sampling_Period
		{
			// Token: 0x06000A92 RID: 2706 RVA: 0x0002BC2B File Offset: 0x00029E2B
			get;
			// Token: 0x06000A93 RID: 2707 RVA: 0x0002BC33 File Offset: 0x00029E33
			set;
		}
	}
}
